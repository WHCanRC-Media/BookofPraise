melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d4 d4 fis4 fis4 a2 a2 b4 b8 b4 b4 a2 fis2 \break

  % Line 2
  a4 a8 a4 a4 r8 d4 d4 cis4 a4 e4 a4 b4 a8 a1 \break

  % Line 3
  d,4 d4 fis4 fis4 a4 a4 a2 b4 b8 b4 b4 a2 a2 \break

  % Line 4
  d2 a4 a4 b2 fis2 g4 e4 e4 d8 d1 \bar "|."
}
