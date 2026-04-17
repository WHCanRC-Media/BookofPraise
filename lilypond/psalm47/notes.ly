melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 fis2 d4 e4 fis4 g2 fis4 e4 e4 d2 r2 \break

  % Line 2
  a'2 a2 b4 cis4 d4 a2 b4 g4 a4 fis2 r2 \break

  % Line 3
  d2 e2 fis4 g4 fis4 e2 d4 d4 cis4 d2 r2 \break

  % Line 4
  a'2 b2 a4 g4 fis4 fis2 e4 fis4 g4 a2 r2 \break

  % Line 5
  d2 cis2 b4 b4 a4 b2 a4 g4 fis4 e2 r2 \break

  % Line 6
  d2 fis2 a4 g4 fis4 e2 d4 d4 cis4 d1 \bar "|."
}
