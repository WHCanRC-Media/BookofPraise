melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 b4 cis4 d2 d2 cis4 b4 cis4 d4 b2 a2 r2 \break

  % Line 2
  a2 a4 g4 fis2 fis2 fis4 e4 fis4 g4 e2 d2 \break

  % Line 3
  r4 a'4 b2 d2 cis2 b2 cis4 d4 b4 b4 a2 r2 \break

  % Line 4
  d2 d4 cis4 b2 a2 a4 b4 a4 g4 fis2 e2 r2 \break

  % Line 5
  e2 a4 a4 b2 a2 a4 a4 b4 cis4 d2 cis2 \break

  % Line 6
  r4 a4 b2 d2 cis2 b2 cis4 d4 b4 b4 a1 \bar "|."
}
