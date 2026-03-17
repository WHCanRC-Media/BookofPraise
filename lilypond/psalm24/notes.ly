melody = \relative c' {
  \clef treble
  \key d \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 e2 b'2 g2 a2 a4 g4 fis2 e2 r2 \break

  % Line 2
  e2 g4 a4 b4 d4 cis4 cis4 b2 r2 \break

  % Line 3
  b2 cis4 b4 a4 g4 fis4 e4 fis2 e2 r2 \break

  % Line 4
  g2 g4 g4 a2 g4 fis4 e2 d2 r2 \break

  % Line 5
  e2 b'4 b4 g4 b4 a2 g2 fis2 r2 \break

  % Line 6
  b2 a4 g4 fis4 e4 g4 a4 fis2 \bar "|."
}
